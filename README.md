# _[f]link_
###### _a minimalist self-hosted file-uploader for friends & family_
&nbsp;

![Screengrab of the login screen][Login]

![Screengrab of the ui screen][Ui]

##### Installation
In order to build the project, you need to install:
* make
* rsync
* cargo & rustup with a recent nightly toolchain
* compass
* diesel-cli

It is recommended to install *diesel-cli* via cargo:
```sh
cargo install diesel_cli --force --no-default-features --features sqlite
```
When all build-dependencies are installed, run:
```sh
$ git clone https://github.com/4nickel/flink
$ cd flink
$ make
$ make service
```
Once the service is running, navigate to ```http://localhost:8000```.

##### User Management
You have to add users manually using a command-line interface - flink doesn't provide an admin interface. You can find the ```flink``` binary in ```server/target/release```.
```sh
$ flink user add $username $password
$ flink user del $username
```

##### Security
Rocket doesn't support SSL yet, so you'll have to run this upstream of an appropriate reverse-proxy server like nginx or apache. If you don't know what that means please make sure you do before running this service in the wild.

##### Powered by..
*  [Rocket] - A simple, fast and secure framework for writing web-services in Rust
*  [Diesel] - A safe, extensible ORM and query-builder for Rust
*  [AngularJs] - Google's Javascript framework for extending HTML
*  [Compass] - An open-source CSS authoring framework

  [Login]: <https://raw.githubusercontent.com/4nickel/flink/master/images/login-screen.png> "Login screen"
  [Ui]: <https://raw.githubusercontent.com/4nickel/flink/master/images/ui-screen.png> "User interface"
  [Diesel]: <https://diesel.rs>
  [Rocket]: <https://rocket.rs>
  [AngularJs]: <https://angularjs.org>
  [Compass]: <https://compass-style.org>
