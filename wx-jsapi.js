/// Create wx_api namespace
(function (ns, wx) {
    ns.config = function (options) {
        wx.config(options);
        return new Promise((resolve, reject) => {
            wx.ready(resolve);
            wx.error(reject);
        });
    };

    ns.checkJsApi = function (apiList) {
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

    ns.chooseImage = function (options) {
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

    ns.uploadImage = function (options) {
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
        wx.uploadImage(options);
        return r;
    };
})((window.wx_api = window.wx_api || {}), wx);
