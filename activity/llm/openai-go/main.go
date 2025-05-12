package main

import "fmt"
import "go.bytecodealliance.org/cm"
import logbindings "github.com/obeli-sk/demo-stargazers/openai-go/gen/obelisk/log/log"
import llmbindings "github.com/obeli-sk/demo-stargazers/openai-go/gen/stargazers/llm/llm"


func myHardcodedLLMResponse(userPrompt string, settingsJSON string) (result cm.Result[string, string, string]) {
	fmt.Printf("[WASM Guest llm.Respond] Received prompt: \"%s\"\n", userPrompt)
	fmt.Printf("[WASM Guest llm.Respond] Received settings: \"%s\"\n", settingsJSON)
	logbindings.Trace("Hello")
	hardcodedResponse := "This is a hardcoded response from the Wasm module. Prompt and settings were received."
	return cm.OK[cm.Result[string, string, string], string /* Shape */, string /* OK */, string /* Err */](hardcodedResponse)
}

func init() {
	// stargazers:llm/llm.respond
	llmbindings.Exports.Respond = myHardcodedLLMResponse
}

func main() {
	panic("wasi:cli/run@0.2.0.run is exported only because of current TinyGo limitation")
}
