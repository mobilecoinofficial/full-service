#!/usr/bin/env python

from distutils.core import setup

setup(
    name='mobilecoin',
    version='0.1',
    description='Command line client for MobileCoin.',
    author='Christian Oudard',
    author_email='christian@christianoudard.com',
    url='https://github.com/christian-oudard/mobilecoin-python-cli',
    packages=['mobilecoin'],
    scripts=['bin/mobcli'],
    install_requires=[
        'requests',
    ],
)
