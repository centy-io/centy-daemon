# Docker testing

We want to have e2e testing in a Docker container, the testing will compile the daemon code and the cli and will test out all the actions and all the combinations, the testing will be managed by vitest and each test will create a project and init it via the cli, and after each command will take a snapshot of the file system of the project it created
