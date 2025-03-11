(function() {
  var form = document.getElementById('form-credential');

  form.addEventListener('submit', function(event) {
    event.preventDefault();
    var pwd_confirm = document.getElementById('pwd_confirm').value;

    if (event.target.password.value !== pwd_confirm) {
      alert('Passwords do not match!');
      return false;
    }
  });
})();
