(function() {
  var bootMeta = document
    .querySelector("meta[name='boot_time']")
    .getAttribute("content");

  setInterval(function() {
    var now = new Date();
    var elapsed = new Date(now - parseInt(bootMeta) * 1000);

    var readable = (elapsed.getUTCDate() - 1) + ' days '
      + elapsed.getUTCHours() + ' hours '
      + elapsed.getUTCMinutes() + ' minutes '
      + elapsed.getUTCSeconds() + ' seconds';

    document.getElementById("uptime").innerHTML = readable;
  }, 1000);
})();
