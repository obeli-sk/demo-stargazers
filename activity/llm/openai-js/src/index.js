import axios from 'axios';
import { getEnvironment } from 'wasi:cli/environment@0.2.3';

const ENV_OPENAI_API_KEY = "OPENAI_API_KEY";
const ENV_OPENAI_API_BASE_URL = "OPENAI_API_BASE_URL";
const DEFAULT_OPENAI_BASE_URL = "https://api.openai.com";

export const llm = {
    async respond(userPrompt, settingsString) {
        console.log("Responding to", userPrompt, settingsString);
        // TODO: Switch to `process.env[ENV_OPENAI_API_KEY]` when https://github.com/bytecodealliance/ComponentizeJS/issues/190 is resolved.
        const envMap = new Map(getEnvironment());
        const apiKey = envMap.get(ENV_OPENAI_API_KEY);
        const baseUrl = envMap.get(ENV_OPENAI_API_BASE_URL) || DEFAULT_OPENAI_BASE_URL;

        if (!apiKey) {
            throw `${ENV_OPENAI_API_KEY} must be set as an environment variable or passed in options`;
        }

        let settings;
        try {
            settings = JSON.parse(settingsString);
        } catch (e) {
            throw `'settings_json' must be parseable: ${e.message}`;
        }

        const initialMessages = settings.messages || [];
        if (!Array.isArray(initialMessages)) {
            throw "'settings.messages' must be an array if provided.";
        }

        const messages = [
            ...initialMessages,
            {
                role: "user", // Corresponds to Role::User
                content: userPrompt,
            },
        ];

        const requestBody = {
            model: settings.model,
            messages: messages,
            max_tokens: settings.max_tokens,
        };

        let response;
        try {
            response = await axios.post(
                `${baseUrl}/v1/chat/completions`,
                requestBody,
                {
                    headers: {
                        "Authorization": `Bearer ${apiKey}`,
                        "Content-Type": "application/json",
                    },
                }
            );
        } catch (error) {
            if (error.response) {
                throw `API Error: ${error.response.status} - ${error.response.statusText}. Details: ${JSON.stringify(error.response.data)}`;
            } else if (error.request) {
                throw `Network Error: No response received. ${error.message}`;
            } else {
                throw `Axios Error: ${error.message}`;
            }
        }

        if (response.status !== 200) {
            throw `Unexpected status code: ${response.status}`;
        }

        const responseData = response.data;

        if (responseData.choices && responseData.choices.length > 0 && responseData.choices[0].message) {
            return responseData.choices[0].message.content;
        }

        throw "No response content from OpenAI or choices array is malformed.";
    }
}
