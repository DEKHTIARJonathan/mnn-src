"""Normalize FindCUDA's split --std argument for sccache's nvcc parser."""
import os
import sys

command = sys.argv[1:3]
arguments = iter(sys.argv[3:])
for argument in arguments:
    if argument == '--std':
        value = next(arguments, None)
        argument = '--std' if value is None else '--std=' + value
    command.append(argument)
os.execv(command[0], command)
