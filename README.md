# _[f]link_
###### _a minimalist self-hosted file-uploader for friends & family_
&nbsp;

![Screengrab of the login screen][Login]

![Screengrab of the ui screen][Ui]

##### Installation
In order to build the project, you need to install make, rsync, cargo, a recent nightly toolchain, and the compass CSS framework. Then run the commands below.
```sh
$ git clone https://github.com/4nickel/flink
$ cd flink
$ make
$ make launch
```
Once the server is running, navigate to ```http://localhost:8000```.

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
