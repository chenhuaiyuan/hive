(function () {
  return {
    is_ipv4: true,
    addr: "127.0.0.1",
    port: 3000,
    exception_fn: () => {},
    serve_fn: () => {},
    bind: function(addr, port) {
      this.addr = addr;
      this.port = port;
      return this;
    },
    ipv6: function() {
      this.is_ipv4 = false;
      return this;
    },
    exception: function(func) {
      this.exception_fn = func;
      return this;
    },
    serve: function(func) {
      this.serve_fn = func;
      return this;
    },
    run: function() {
      return {
        addr: this.addr,
        port: this.port,
        exception: this.exception_fn,
        serve: this.serve_fn,
        is_ipv4: this.is_ipv4
      };
    }
  }
})()
