package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"

	"go.bytecodealliance.org/cm"
	"github.com/obeli-sk/demo-stargazers/openai-go/wasihttp"
	logbindings "github.com/obeli-sk/demo-stargazers/openai-go/gen/obelisk/log/log"
	llmbindings "github.com/obeli-sk/demo-stargazers/openai-go/gen/stargazers/llm/llm"
)

const openAIEnv = "OPENAI_API_KEY"


type Role string

const (
	RoleUser      Role = "user"
	RoleAssistant Role = "assistant"
	RoleSystem    Role = "system"
)

// Message is one chat message.
type Message struct {
	Role    Role   `json:"role"`
	Content string `json:"content"`
}

// Payload for the OpenAI API.
type OpenAIRequest struct {
	Model     string    `json:"model"`
	Messages  []Message `json:"messages"`
	MaxTokens int       `json:"max_tokens"`
}

// choice and response structs for parsing.
type choice struct {
	Message struct {
		Content string `json:"content"`
	} `json:"message"`
}
type OpenAIResponse struct {
	Choices []choice `json:"choices"`
}

// Settings is what the host passes in.
type Settings struct {
	Model     string    `json:"model"`
	Messages  []Message `json:"messages"`
	MaxTokens int       `json:"max_tokens"`
}

var (
	wasiTransport = &wasihttp.Transport{
		// Not setting ConnectTimeout, timeouts are handled by Obelisk
	}
	httpClient    = &http.Client{Transport: wasiTransport}
)

func respond(userPrompt string, settingsJSON string) (result cm.Result[string, string, string]) {
	// 1. API key
	apiKey := os.Getenv(openAIEnv)
	if apiKey == "" {
		return cm.Err[cm.Result[string, string, string]](
			fmt.Sprintf("%s must be set as an environment variable", openAIEnv),
		)
	}

	// 2. Parse settings
	var settings Settings
	if err := json.Unmarshal([]byte(settingsJSON), &settings); err != nil {
		return cm.Err[cm.Result[string, string, string]](
			fmt.Sprintf("invalid settings JSON: %v", err),
		)
	}

	// 3. Append user message
	settings.Messages = append(settings.Messages, Message{
		Role:    RoleUser,
		Content: userPrompt,
	})

	// 4. Build request body
	reqBody := OpenAIRequest{
		Model:     settings.Model,
		Messages:  settings.Messages,
		MaxTokens: settings.MaxTokens,
	}
	rawReq, err := json.Marshal(reqBody)
	if err != nil {
		return cm.Err[cm.Result[string, string, string]](
			fmt.Sprintf("failed to serialize request: %v", err),
		)
	}
	fmt.Println("OpenAI request:", string(rawReq))

	// 5. Do HTTP POST via wasihttp
	req, err := http.NewRequest(http.MethodPost, "https://api.openai.com/v1/chat/completions", bytes.NewReader(rawReq))
	if err != nil {
		return cm.Err[cm.Result[string, string, string]](
			fmt.Sprintf("failed to create request: %v", err),
		)
	}
	req.Header.Set("Authorization", "Bearer "+apiKey)
	req.Header.Set("Content-Type", "application/json")
	req.ContentLength = int64(len(rawReq))

	resp, err := httpClient.Do(req)
	if err != nil {
		return cm.Err[cm.Result[string, string, string]](
			fmt.Sprintf("failed to make outbound request: %v", err),
		)
	}
	defer resp.Body.Close()
	rawResp, _ := io.ReadAll(resp.Body)

	// 6. Check status
	if resp.StatusCode != http.StatusOK {
		fmt.Println("OpenAI error response:", string(rawResp))
		return cm.Err[cm.Result[string, string, string]](
			fmt.Sprintf("unexpected status code: %d", resp.StatusCode),
		)
	}

	// 7. Read & parse response
	var apiResp OpenAIResponse
	if err := json.Unmarshal(rawResp, &apiResp); err != nil {
		return cm.Err[cm.Result[string, string, string]](
			fmt.Sprintf("failed to parse response JSON: %v", err),
		)
	}

	// 8. Return first choice or error
	if len(apiResp.Choices) > 0 {
		logbindings.Debug("HTTP roundtrip succeeded")
		return cm.OK[cm.Result[string, string, string]](apiResp.Choices[0].Message.Content)
	}
	return cm.Err[cm.Result[string, string, string]]("no response from OpenAI")
}

func init() {
	// stargazers:llm/llm.respond
	llmbindings.Exports.Respond = respond
}

func main() {
	logbindings.Error("wasi:cli/run@0.2.0.run is exported only because of current TinyGo limitation")
	panic("wasi:cli/run@0.2.0.run is exported only because of current TinyGo limitation")
}
