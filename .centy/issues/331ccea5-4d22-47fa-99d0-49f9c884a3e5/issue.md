# Title generation

As part of the llm integration in issue cf64f3d7-832f-4ba9-831b-2589a1c8e790

We want to make the issue title optional, if the user does not send a title it will auto generate via the integration with the LLM, if there is no title and no LLM integrated send an error, if the LLM is not available send a relevant error as well.
