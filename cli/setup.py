#!/usr/bin/env python

from distutils.core import setup

setup(
    name='mobilecoin-cli',
    version='1.0',
    description='Command line client for MobileCoin.',
    author='Christian Oudard',
    author_email='christian@mobilecoin.com',
    url='https://github.com/mobilecoinofficial/full-service/tree/main/cli',
    packages=['mobilecoin'],
    scripts=['bin/mobcli'],
    install_requires=[],
    data_files=[
        ('scripts', ['mc_env.sh', 'install.sh']),
    ],
)
