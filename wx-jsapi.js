/// Create wx_api namespace
(function (ns, wx) {
    ns.config = function config(options) {
        wx.config(options);
        return new Promise((resolve, reject) => {
            wx.ready(resolve);
            wx.error(reject);
        });
    };

    ns.checkJsApi = function checkJsApi(apiList) {
        let saved_resolve, saved_reject;
        let r = new Promise((resolve, reject) => {
            saved_resolve = resolve;
            saved_reject = reject;
        });
        wx.checkJsApi({
            jsApiList: apiList,
            success: saved_resolve,
            fail: saved_reject,
        });
        return r;
    };

    ns.chooseImage = function chooseImage(options) {
        let saved_resolve, saved_reject;
        let r = new Promise((resolve, reject) => {
            saved_resolve = resolve;
            saved_reject = reject;
        });
        options = {
            success: saved_resolve,
            fail: saved_reject,
            ...options,
        };
        wx.chooseImage(options);
        return r;
    };
})((window.wx_api = window.wx_api || {}), wx);
