# fs-integration

These scrips use Python Poetry Env/Package manager

## To run locally

1. Setup poetry (see sections below for more details)

```
curl -sSL https://install.python-poetry.org | python3 -
poetry install
```

2. Add mnemonics to `test_config.json` (or create another configuration file and pass as another parameter when running the test)
   - Don't worry, `.gitignore` will prevent you from accidentally checking in your secrets

3. Run the test

```
poetry run python3 basic.py
```



## Install Dependencies

## Activate environment and run

You can activate the environment for this and following commands:

```
poetry shell
python3 ./<script>
```

OR

```
poetry run ./<script>
```

## Install poetry

This will install `poetry` for your local user

```bash
curl -sSL https://install.python-poetry.org | python3 -
```

You may need to add `poetry` to your `PATH` in your shell profile rc file.

```bash
export PATH="${HOME}/.local/bin:${PATH}"
```

## Set up new project

For these simple projects, lets not use the full poetry directory structure. Just create a directory and `init`. This will walk you through the `pyproject.toml` creation.

```bash
poetry init
```

Remove the default directory project directory in the `pyproject.toml`, we're not using subfolders for this simple project.

```
{include = "fs_integration"}
```

## Use existing poetry configuration
To read and install dependencies from current project's `pyproject.toml`: 
```
poetry install
```

## VSCode hints.

**Activate Poetry**

See above to activate poetry before opening vscode.

**Fix local package intellisense.**

If VSCode can't find `fullservice` add the path with .env file and settings config.

Add to `.vscode/settings.json`

```json
{
    "python.envFile": "${workspaceFolder}/.vscode/vscode.env"
}
```

Add a `.vscode/vscode.env` file with `PYTHONPATH` defined.

```bash
PYTHONPATH=${PYTHONPATH}:python-library
```
