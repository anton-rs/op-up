## Troubleshooting

- If you are getting some "permission denied" errors, it's likely that you need to change the Docker permissions on your machine. See [this guide](https://docs.docker.com/engine/install/linux-postinstall/) for more details.
- If you are getting the error: `Failed to install dependencies: error hardhat@2.9.6: The engine "node" is incompatible with this module.` you need to switch your NodeJS version to <=16. If you are using `nvm`, you can do so by running `nvm install 16.16.0 && nvm use 16.16.0`.
- If you are on MacOS with Apple Silicon chip and you've installed python3 via Homebrew, you might run into this error: `env: python: No such file or directory. make: *** [Release/leveldb.a] Error 127`. To fix this, you need to create a symlink to the python3 binary like so: `sudo ln -s /Library/Developer/CommandLineTools/usr/bin/python3 /usr/local/bin/python`.
- If you run into an issue while building the Hardhat bedrock project, please set your node version to `16.16.0`. For instance if you are using `nvm`, you can do so by running `nvm install 16.16.0 && nvm use 16.16.0`. See [this issue](https://github.com/ethereum-optimism/optimism#3087) for more details.


