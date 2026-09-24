# Use your own field names

The published upload endpoint accepts an optional `template_id`. The [official docs](https://formalini.com/docs) say it makes extracted fields come back with your own names. Create and manage templates in the Formalini app, then pass the selected template ID when uploading page images.

The public docs do not specify a template creation API, schema definition format, or validation rule language. This repository does not invent one. Before using a template in production, upload a representative page in a dedicated workspace, inspect the returned fields, and handle absent or uncertain values in your application.
