#!/usr/bin/env python

from distutils.core import setup

setup(
    name='mobilecoin-python-utils',
    version='0.1',
    description='Python utility functions for working with MobileCoin addresses and receipts.',
    author='Eran Rundstein',
    author_email='eran@mobilecoin.com',
    url='https://github.com/mobilecoinofficial/full-service/tree/main/python-utils',
    packages=['mc_util'],
    install_requires=['base58', 'protobuf'],
)
