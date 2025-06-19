(function() {
  var modalButtons = document.getElementsByClassName('modal-confirm');

  for (var i = 0; i < modalButtons.length; i++) {
    modalButtons[i].addEventListener('click', function(event) {
      event.preventDefault();

      Swal.fire({
        title: event.target.getAttribute('data-modal-message'),
        icon: 'warning',
        showCancelButton: true,
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        customClass: {
          confirmButton: 'btn-swal btn-confirm',
          cancelButton: 'btn-swal btn-cancel',
        },
        theme: 'auto',
      }).then(function(result) {
        if (!result.isConfirmed) {
          return;
        }

        window.location.href = event.target.href;
      });
    });
  }
})();
