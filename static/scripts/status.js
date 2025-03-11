(function() {
  var bootMeta = document
    .querySelector("meta[name='boot_time']")
    .getAttribute('content');

  var getUnit = function(value, unitSingular) {
    return value > 1 ? unitSingular.concat('s') : unitSingular;
  };

  setInterval(function() {
    var now = new Date();
    var elapsed = new Date(now - parseInt(bootMeta) * 1000);

    var readableDate = elapsed.getUTCDate() - 1;
    var readableHours = elapsed.getUTCHours();
    var readableMinutes = elapsed.getUTCMinutes();
    var readableSeconds = elapsed.getUTCSeconds();

    var readable = [
      readableDate,
      getUnit(readableDate, 'day'),
      readableHours,
      getUnit(readableHours, 'hour'),
      readableMinutes,
      getUnit(readableMinutes, 'minute'),
      readableSeconds,
      getUnit(readableSeconds, 'second'),
    ].join(' ');

    document.getElementById('uptime').innerHTML = readable;
  }, 1000);
})();
