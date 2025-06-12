var swalConfirm = function(anchor, message) {
  Swal.fire({
    title: message,
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

    window.location.href = anchor.href;
  });

  return false;
};
