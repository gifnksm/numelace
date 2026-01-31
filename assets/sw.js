var cacheName = "numelace-pwa-v1";
var filesToCache = ["./", "./index.html"];

/* Start the service worker and cache all of the app's content */
self.addEventListener("install", function (e) {
    e.waitUntil(
        caches.open(cacheName).then(function (cache) {
            return cache.addAll(filesToCache);
        }),
    );
});

/* Remove old caches on activate */
self.addEventListener("activate", function (e) {
    e.waitUntil(
        caches.keys().then(function (keys) {
            return Promise.all(
                keys.map(function (key) {
                    if (key !== cacheName) {
                        return caches.delete(key);
                    }
                }),
            );
        }),
    );
});

/* Serve cached content when offline */
self.addEventListener("fetch", function (e) {
    var requestUrl = new URL(e.request.url);

    if (
        requestUrl.pathname.endsWith("/index.html") ||
        e.request.mode === "navigate"
    ) {
        e.respondWith(
            fetch(e.request)
                .then(function (response) {
                    return caches.open(cacheName).then(function (cache) {
                        cache.put(e.request, response.clone());
                        return response;
                    });
                })
                .catch(function () {
                    return caches.match(e.request);
                }),
        );
        return;
    }

    e.respondWith(
        caches.match(e.request).then(function (response) {
            if (response) {
                return response;
            }

            return fetch(e.request).then(function (networkResponse) {
                return caches.open(cacheName).then(function (cache) {
                    cache.put(e.request, networkResponse.clone());
                    return networkResponse;
                });
            });
        }),
    );
});
