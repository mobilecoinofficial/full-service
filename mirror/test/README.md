# Testing for Full Service Mirror

This directory contains a shell script + associated files for testing a full service mirror release.

This can be run with varying degress of automation. Either with docker, or by running run.sh with a binary you downloaded yourself

## Test with Docker

To test a release, change the Dockerfile mirversion to point to the release of the fullservicemirror that you want to test. Then call the following:
```sh
docker built -t fullservice-mirror-test .
```
```sh
docker run fullservice-mirror-test
```

## Test without Docker using run.sh

Download the release and unzip it to the same directory that run.sh is in, then you can call run.sh and it should start LVN, Full Service, Full Service Mirror, and run all the tests against it.
