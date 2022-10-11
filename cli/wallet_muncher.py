import urllib.request
import re
import json

# This script takes in a Full Service URL, parses it, and converts it into valid JSON.
# Then prints function definitions which can be piped into a file and used in fullservice.py 
# Run it through black afterwards. 

lines = [
    line
    for line in (
        urllib.request.urlopen("https://readonly-fs-mainnet.mobilecoin.com/wallet/v2") # Full Service help page to parse, either /wallet or /wallet/v2
        .read()
        .decode()
        .split("\n\n")[3:]  # we want everything after the first three lines (or so)
    )
    if line
]


def get_classes(lines):
    # the unique set of things that start with capital letters and aren't the special cases None or V1
    return set(
        [x for x in re.findall("[A-Z]\w+", "\n".join(lines)) if x not in ["None", "V1"]]
    )


def return_method_and_parameters(current_line):
    current_line = current_line.replace(
        "JsonU64(0)", "0"
    )  # v1 api uses json floats for fees
    useless_typing = get_classes([current_line])
    arguments = re.findall(pattern="[\w_]+: ", string=current_line)
    # sort arguments by length, longest to shortest
    sorted_arguments = list(sorted(arguments, key=lambda x: -len(x)))
    if current_line.count(" ") > 1:  # if there's no spaces there are no arguments
        method, parameters = (
            [
                current_line := current_line.replace(x, f'"{x[:-1]}": ').replace(
                    ':":', '":'
                )
                for x in sorted_arguments
            ][-1]
            .replace("None", "null")  # make it json
            .split(" ", 1)  # method is the first word
        )
    elif current_line:
        method, parameters = current_line, "{}"
    parameters = parameters.replace("V1", '"V1"')  # make sure V1 is in quotes
    # mutates parameters to remove useless typing items once at a time
    [parameters := parameters.replace(to_remove, "") for to_remove in useless_typing]
    return method, json.loads(parameters)


methods_and_parameters = [return_method_and_parameters(line) for line in lines]
for method, parameters in methods_and_parameters:
    if parameters and list(parameters.keys())[0] == "account_id":
        empty_str = '""'
        all_args = ", ".join(
            [
                f"{key} = {value or empty_str}"
                for (key, value) in list(parameters.items())[1:]
            ]
        )
        rpc_params = ", " + ", ".join(
            [f'"{key}": {key}' for (key, value) in list(parameters.items())[1:]]
        )
        stub = f"""async def {method}(self, account_id, {all_args}):"""
        stub += f"""\n\t return await self.req({{"method": "{method}", "params": {{"account_id": account_id"""
        stub += rpc_params
        stub += "}})"
        print(stub)

for method, parameters in methods_and_parameters:
    if not parameters:
        empty_str = '""'
        stub = f"""async def {method}(self):"""
        stub += (
            f"""\n\t return await self.req({{"method": "{method}", "params": {{}}}})"""
        )
        print(stub)

for method, parameters in methods_and_parameters:
    if parameters and list(parameters.keys())[0] != "account_id":
        empty_str = '""'
        all_args = ", ".join(
            [
                f"{key} = {value or empty_str}"
                for (key, value) in list(parameters.items())
            ]
        )
        rpc_params = ", ".join(
            [f'"{key}": {key}' for (key, value) in list(parameters.items())]
        )
        stub = f"""async def {method}(self, {all_args}):"""
        stub += f"""\n\t return await self.req({{"method": "{method}", "params": {{"""
        stub += rpc_params
        stub += "}})"
        print(stub)