# API Key

You can add an optional API key to full service by adding a `.env` file to the root of this repo. The variable you need
to set is: `MC_API_KEY="<api key of your choosing>"`. If you set this env var, you must provide the `X-API-KEY` header
in your requests to full-service.
