//
// Web frontend for the flink server.
//
// Author: Felix
// Date: 2019
//

var app = angular.module('flink', ['ngRoute', 'ngResource', 'ngAnimate']);

//{{{ Utility

var elementScope = function(element) {
    return angular.element(element).scope();
};

var onChange = function(element) {
    elementScope(element).data.onChange(element)
};

var templateDir = 'app';
var imageDir = 'img';

// var domain  = 'https://flink.com'
var domain  = 'localhost:8000';
var apiUrl  = '/api';
var apiFile = apiUrl + '/file';
var apiAuth = apiUrl + '/auth';

var humanReadable = function(bytes) {
    var kb = 1024;
    var mb = kb*1024;
    var gb = mb*1024;
    var tb = gb*1024;
    if(bytes >= tb) return [bytes/tb, 'tb']
    if(bytes >= gb) return [bytes/gb, 'gb']
    if(bytes >= mb) return [bytes/mb, 'mb']
    if(bytes >= kb) return [bytes/kb, 'kb']
    return [bytes, 'b']
}

//}}}
//{{{ Factories

app.factory('File', function($resource) {

    var Resource = $resource(apiFile + '/:key',
        {
            key: '@key',
        },{
            delete: { method: 'DELETE' },
        }
    );

    function File(module, data) {
        this.mod = module;
        this.api = new Resource(data);
        this.size = humanReadable(this.api.bytes);
        this.show = true;
    }

    File.res = Resource;

    File.prototype.relLink = function() {
        return '/f/' + this.api.key;
    };

    File.prototype.absLink = function() {
        return domain + this.relLink();
    };

    File.prototype.delete = function() {
        var context = this;
        this.api.$delete().then(
            function(success) {
                var index = context.mod.fileList.indexOf(context);
                if(index > -1) {
                    context.mod.fileList.splice(index, 1);
                } else {
                    context.mod.updateFileList();
                }
            },
            function(failure) {
            },
        );
    };

    File.prototype.download = function() {
        this.mod.sendDownload(this.api.key);
    };

    return File;
});

//}}}
//{{{ Services
//{{{ History

app.service('History', function(File, Index, $http) {

    this.fileList = [];
    this.show = false;
    this.expand = true;

    this.clear = function() {
        this.fileList = [];
        this.show = false;
    };

    this.push = function(file) {
        console.log(file);
        this.fileList.push(file);
        this.show = true;
    }

    this.toggleExpand = function() {
        this.expand = !this.expand;
    }
});

//}}}
//{{{ Upload

app.service('Upload', function(File, Index, History, $http) {

    this.meta = 'w';
    this.name = '';
    this.file = undefined;
    this.show = false;
    this.expand = true;

    // By default, angularjs doesn't bind
    // file selection inputs. We could use
    // a directive, instead.
    this.onChange = function(element) {
        if(element.files.length > 0){
            console.log('[Upload] setting file')
            this.updateFileSelection(element.files[0]);
            elementScope(element).$apply();
        } else {
            console.log('[Upload] nothing selected')
        }
    }

    this.updateFileSelection = function(file) {
        this.file = file;
        this.name = file.name;
        this.show = true;
        angular.forEach(angular.element(document.querySelector('#BrowserName')),
            function(element) {
                angular.element(element).val(file.name);
            }
        );
    }

    // HTML file selection inputs are cumbersome
    // to use with angular. After a successfull
    // upload, we have to clear to clear the value
    // by hand.
    this.clearAllFileInputs = function() {
        angular.forEach(
            angular.element(document.querySelector('input[type="file"]')),
            function(element) {
                angular.element(element).val(null);
            }
        );
    }

    this.clearUpload = function() {
        this.meta = 'w';
        this.name = '';
        this.file = undefined;
        this.show = false;
        this.clearAllFileInputs();
    }

    this.sendUpload = function(successFn, failureFn) {
        if(this.file == undefined) {
            console.log('[Upload] file is undefined');
            return
        } else {
            console.log('[Upload] uploading file ' + JSON.stringify(this.file));
        }
        var context = this;

        var multipart = new FormData();
        multipart.append('name', this.name);
        multipart.append('file', this.file);
        multipart.append('meta', this.meta);
        var request = { method: 'POST', url: apiFile, headers: {'Content-Type': undefined}, data: multipart };
        $http(request).then(
            function(success){
                console.log('[Upload] file upload success');
                context.clearUpload();

                var file = Index.push(success.data);
                History.push(file);
                successFn(success);
            },
            function(failure){
                console.log('[Upload] file upload failure');
                failureFn(failure);
            }
        );
    }

    this.toggleExpand = function() {
        this.expand = !this.expand;
    }

    this.acceptUpload = function() {
        this.sendUpload(
            function(success) { },
            function(failure) { },
        );
    }

    this.cancelUpload = function() {
        this.clearUpload();
    }

});

//}}}
//{{{ Index

app.service('Index', function(File, Login, $location, $http) {
    console.log('[Index] initializing')

    this.File = File;
    this.Login = Login;

    this.show = true;
    this.fileList = [];
    this.userInfo = {
        storage: 0,
        link: {
            up: 0,
            down: 0,
        },
    };

    this.push = function(file) {
        var created = new File(this, file);
        this.fileList.push(created);
        this.userInfo.storage += file.bytes;
        this.userInfo.link.down += file.downloads;
        this.userInfo.link.up = this.fileList.length;
        this.show = true;
        return created;
    };

    this.sendDownload = function(key) {
        var file = new File(this, {});
        file.api.$get({ key: key }).then(
            function(success) { },
            function(failure) { }
        );
    };

    this.clearFileList = function() {
        console.log('[Index] clearing file list')
        this.fileList = [];
        this.userInfo.storage = 0;
        this.userInfo.link.down = 0;
        this.userInfo.link.up = 0;
    };

    this.forbidden = function() {
        console.log('[Index] access forbidden');
        this.show = false;
        this.Login.clearUser();
        $location.path('/login');
    };

    this.updateFileList = function() {
        console.log('[Index] updating file list')
        var context = this;

        this.File.res.query().$promise.then(
            function(success) {
                console.log('[Index] access granted')
                console.log('[Index] ' + success.length + ' items found');
                context.clearFileList();
                angular.forEach(success, function(v, k) { context.push(v) });
            },
            function(failure) {
                console.log('[Index] access denied');
                if(failure.status == 403) { context.forbidden(); }
            },
        );
    };

    this.updateUserName = function() {
        var context = this;

        if(this.Login.user.name != undefined)
            return;

        this.Login.queryUser(
            function(success) {
                console.log('[Index] access granted')
                context.show = true;
            },
            function(failure) {
                console.log('[Index] access denied')
                if(failure.status == 403) { context.forbidden(); }
            },
        );
    };
});

//}}}
//{{{ Login

app.service('Login', function($http, $location) {
    console.log('[Login] initializing')
    this.username = '';
    this.password = '';
    this.expand = false;

    this.user = {
        name: undefined,
        auth: false,
    };

    this.sendLogout = function() {
        console.log('[Login] sending logout request');
        var context = this;

        var request = { method: 'DELETE', url: apiAuth + '/login' };
        $http(request).then(
            function(success) {
                console.log('[Login] logout success');
                context.onChange();
                $location.path('/login');
            },
            function(failure) {
                console.log('[Login] logout failure');
            }
        );
    }

    this.sendLogin = function(username, password) {
        console.log('[Login] sending login request');
        var context = this;

        var request = { method: 'POST', url: apiAuth + '/login', data: { username: username, password: password } };
        $http(request).then(
            function(success) {
                console.log('[Login] login success');
                context.user.name = success.data.name;
                context.user.auth = true;
                context.username = '';
                context.password = '';
                $location.path('/');
            },
            function(failure) {
                console.log('[Login] login failure');
            }
        );
    }

    this.setUser = function(data) {
        this.user.name = data.name;
        this.user.auth = true;
    }

    this.clearUser = function() {
        this.user.name = undefined;
        this.user.auth = false;
    }

    this.queryUser = function(successFn, failureFn) {
        console.log('[Login] sending status query');
        var context = this;

        var request = { method: 'GET', url: apiAuth + '/login' };
        return $http(request).then(
            function(success) {
                console.log('[Login] query success');
                context.setUser(success.data);
                successFn(success);
            },
            function(failure) {
                console.log('[Index] query failure');
                if(failure.status == 403) {
                    context.clearUser();
                }
                failureFn(failure);
            }
        )
    }

    this.acceptLogin = function() {
        this.sendLogin(this.username, this.password);
    };

    this.onChange = function() {
        this.expand = !(this.username == '' && this.password == '');
    };
});

//}}}
//}}}
//{{{ Controllers
//{{{ Login
app.controller('LoginController', function LoginController($scope, $location, Login) {
    console.log('[LoginController] initializing')
    $scope.title = 'login';
    $scope.Login = Login;

    $scope.$on('$routeChangeSuccess', function(scope, next, current){
        console.log('[Login] route changed');
        if($scope.Login.user.auth) {
            $location.path('/');
        }
    });
});

//}}}
//{{{ Index

app.controller('IndexController', function IndexController($scope, Index, Login, Upload, History) {
    console.log('[IndexController] initializing')
    $scope.title = 'upload';
    $scope.Upload = Upload;
    $scope.Login = Login;
    $scope.Index = Index;
    $scope.History = History;

    // Send a logout request
    $scope.logout = function() {
        this.Login.sendLogout();
    }

    // Send a file-download request
    $scope.downloadFile = function(key) {
        this.Upload.sendDownload(key);
    }

    $scope.$on('$routeChangeSuccess', function(scope, next, current){
        console.log('[Index] route changed');
        $scope.Index.updateUserName();
        $scope.Index.updateFileList();
    });
});

//}}}
//}}}
//{{{ Directives
//{{{ Page

app.directive('pageHeader', function() {
    return {
        templateUrl: templateDir + '/page/header.html',
    };
});

app.directive('pageFooter', function() {
    return {
        templateUrl: templateDir + '/page/footer.html',
    };
});

//}}}
//{{{ Widgets

app.directive('iconBubble', function() {
    return {
        templateUrl: templateDir + '/widgets/icon-bubble.html',
        scope: {
            icon: '@',
        },
        transclude: true,
    };
});

//}}}
//{{{ Ui

app.directive('browser', function() {
    return {
        templateUrl: templateDir + '/ui/browser.html',
        scope: {
            data: '=',
        },
    };
});

app.directive('history', function() {
    return {
        templateUrl: templateDir + '/ui/history.html',
        scope: {
            data: '=',
        },
    };
});

app.directive('fileList', function() {
    return {
        templateUrl: templateDir + '/ui/file-list.html',
        scope: {
            data: '=',
        },
    };
});

app.directive('file', function() {
    return {
        templateUrl: templateDir + '/ui/file.html',
        scope: {
            data: '=',
        },
    };
});

app.directive('login', function() {
    return {
        templateUrl: templateDir + '/ui/login.html',
        scope: {
            data: '=',
        },
    };
});

//}}}
//}}}
//{{{ Routing

app.config(function($routeProvider, $httpProvider) {
    $routeProvider
    .when('/', {
        templateUrl: templateDir + '/site/index.html',
        controller: 'IndexController',
    })
    .when('/login', {
        templateUrl: templateDir + '/site/login.html',
        controller: 'LoginController',
    })
    .otherwise({
        templateUrl: templateDir + '/site/index.html',
        controller: 'IndexController',
    });
});

//}}}
