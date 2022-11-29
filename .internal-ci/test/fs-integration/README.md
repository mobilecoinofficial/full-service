# fs-integration

These scrips use Python Poetry Env/Package manager.

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

