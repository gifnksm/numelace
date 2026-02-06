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

    // Navigation and entry point: network-first, update cache on success,
    // fall back to cache when offline so the app still loads.
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

    // Worker assets: network-first to avoid stale protocol mismatches,
    // but still fall back to cache when offline.
    if (
        requestUrl.pathname.endsWith("/numelace-worker.js") ||
        requestUrl.pathname.endsWith("/numelace-worker_bg.wasm") ||
        requestUrl.pathname.endsWith("/numelace-worker-bootstrap.js")
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
