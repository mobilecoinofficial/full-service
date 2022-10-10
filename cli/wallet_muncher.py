import urllib.request
import re
import json

lines = [
    line
    for line in (
        urllib.request.urlopen("http://localhost:9090/wallet/")
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
        print(all_args)
