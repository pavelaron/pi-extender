(function() {
  return;

  var token = document.cookie.replace(/(?:(?:^|.*;\s*)token\s*\=\s*([^;]*).*$)|^.*$/, "$1");

  if (token) {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', '/verify', true);
    xhr.setRequestHeader('Authorization', 'Bearer ' + token);
    xhr.send();
  }
})();
