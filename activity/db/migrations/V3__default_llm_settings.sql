-- A basic prompt. Without `model`, the workflow uses the first model offered by the LLM endpoint.
INSERT INTO llm (id, settings) VALUES (1, '{"messages":[{"role":"system","content":"Write a concise, friendly description of a GitHub user in two or three sentences, based on the JSON provided."}],"max_tokens":200}')
ON CONFLICT (id) DO NOTHING;
