import urllib.request
import re
import json

lines = (
    urllib.request.urlopen("http://localhost:9090/wallet/v2")
    .read()
    .decode()
    .split("\n\n")[3:] # we want everything after the first three lines (or so)
)

# class Postman: # decode the json file exported from postman
#     def main():
#         with open("/Users/zoey/v2.postman_collection.json", "r") as f:
#             data = f.read() 
#             lines = json.loads(data) # convert to string
#             print(lines)


def get_classes(lines):
    # the unique set of things that start with capital letters and aren't the special cases None or V1
    return set(
        [x for x in re.findall("[A-Z]\w+", "\n".join(lines)) if x not in ["None", "V1"]]
    )


def return_method_and_parameters(current_line):
    useless_typing = get_classes([current_line])
    method, parameters = (
        [
            current_line := current_line.replace(x, f'"{x[:-1]}": ')
            for x in re.findall(pattern="[\w_]+:", string=current_line)
        ][-1]
        .replace("None", "null") # make it json
        .split(" ", 1) # method is the first word
    )
    # mutates parameters to remove useless typing items once at a time
    [parameters := parameters.replace(to_remove, "") for to_remove in useless_typing]
    return method, parameters

def get_function_parameters(lines):
    #functions = return_method_and_parameters[0][0].split("(", 1)[1].replace("\n", "")
    return set(
        [x for x in re.findall("[a-z]\w+:", "\n".join(lines)) if x not in ["None", "V1"]]
    )


# current_classes = get_classes(lines)

return_method_and_parameters(lines[7]) # not subscriptable? function...

# seperate paramaters by quotes
# def seperate_parameters():
#     para = pickline() 
#     parameters = str(para).replace(" ", "").split(",")
#     return parameters
    

# if __name__ == "__main__":
#     return_method_and_parameters(lines[7])
#     #Postman.main()
