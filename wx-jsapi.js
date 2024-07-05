/// Create wx_api namespace
(function (ns, wx) {
  ns.config = function (options) {
    wx.config(options);
    return new Promise((resolve, reject) => {
      wx.ready(resolve);
      wx.error(reject);
    });
  };

  function wrap(f) {
    return (options) => {
      let saved_resolve;
      let r = new Promise((resolve) => {
        saved_resolve = resolve;
      });
      wx[f]({
        complete: saved_resolve,
        ...options,
      });
      return r;
    };
  }

  function wx_js_bridge(f) {
    return (options) => {
      let saved_resolve;
      let r = new Promise((resolve) => {
        saved_resolve = resolve;
      });
      WeixinJSBridge.invoke(f, options, saved_resolve);
      return r;
    };
  }

  ns.checkJsApi = wrap("checkJsApi");
  ns.chooseImage = wrap("chooseImage");
  ns.uploadImage = wrap("uploadImage");
  ns.pay = wx_js_bridge("getBrandWCPayRequest");
})((window.wx_api = window.wx_api || {}), wx);
