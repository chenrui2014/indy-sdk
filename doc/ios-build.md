# Setup of IOS build environment.

1. Install [rustup](https://www.rustup.rs) toolchain manager.

2. Install toolchains using command
   > rustup target add aarch64-apple-ios armv7-apple-ios armv7s-apple-ios i386-apple-ios x86_64-apple-ios

3. Install cargo-lipo
   > cargo install cargo-lipo

4. Install required native libraries and utilities
   > brew install libsodium
   > brew install zeromq
   > brew install cmake

5. Setup environment variables:

   > export PKG_CONFIG_ALLOW_CROSS=1
   > export CARGO_INCREMENTAL=1

6. Edit script build-libsovrin-core-ios.sh: set the following variables to fit your environment:
   > export OPENSSL_DIR=/usr/local/Cellar/openssl/1.0.2k
   > export EVERNYM_REPO_KEY=~/Documents/EvernymRepo
   > export LIBSOVRIN_POD_VERSION=0.0.1
    
   OPENSSL_DIR - path to installed openssl library
   EVERNYM_REPO_KEY - path to file with private key to be authorized on deb server
   LIBSOVRIN_POD_VERSION - version of livsovrin-core pod to be built
    
7. Run the script. Validate the output that all goes well.

8. cd Podspec

9. Create directory with name defined in LIBSOVRIN_POD_VERSION
    mkdir LIBSOVRIN_POD_VERSION

10. copy libsovrin-core.podspec.json to that new directory from some previous version

11. edit this json -> change version field to LIBSOVRIN_POD_VERSION

12. add new directory and file inside to git repository.

13. commit to master branch

14. for all projects which using libsovrin-core do not forget to make
     > pod repo update
     > pod install


