package main

import (
	"fmt"
	"io"
	"net/http"
"go.bytecodealliance.org/cm"
"github.com/rajatjindal/wasi-go-sdk/pkg/wasihttp"
logbindings "github.com/obeli-sk/demo-stargazers/openai-go/gen/obelisk/log/log"
llmbindings "github.com/obeli-sk/demo-stargazers/openai-go/gen/stargazers/llm/llm"
"os"
)


func respond(userPrompt string, settingsJSON string) (result cm.Result[string, string, string]) {
	value := os.Getenv("OPENAI_API_KEY")
	fmt.Println("OPENAI_API_KEY:", value)
	
	client := wasihttp.NewClient()
	req, err := http.NewRequest(http.MethodGet, userPrompt, nil)
	if err != nil {
		errorMsg := fmt.Sprintf("failed to create request: %v", err)
		return cm.Err[cm.Result[string, string, string]](errorMsg)
	}

	resp, err := client.Do(req)
	if err != nil {
		errorMsg := fmt.Sprintf("failed to make outbound request: %v", err)
		return cm.Err[cm.Result[string, string, string]](errorMsg)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		errorMsg := fmt.Sprintf("expected status code: %d, got: %d", http.StatusOK, resp.StatusCode)
		return cm.Err[cm.Result[string, string, string]](errorMsg)
	}

	raw, err := io.ReadAll(resp.Body)
	if err != nil {
		errorMsg := fmt.Sprintf("failed to read response body: %v", err)
		return cm.Err[cm.Result[string, string, string]](errorMsg)
	}


	logbindings.Debug("HTTP request succeeded")
	
	return cm.OK[cm.Result[string, string, string]](string(raw))
}

func init() {
	// stargazers:llm/llm.respond
	llmbindings.Exports.Respond = respond
}

func main() {
	panic("wasi:cli/run@0.2.0.run is exported only because of current TinyGo limitation")
}
