// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

const CACHE_KEY = "wasefire v0.1";

const CACHE = [
  "/",
  "index.html",
  "style.css",
  "title.svg",
  "board_components.js",
  "board.js",
  "favicon.png",
  "components/button.svg",
  "components/monochrome_led.svg",
];

async function install() {
  const cache = await self.caches.open(CACHE_KEY);
  await cache.addAll(CACHE);
  await self.skipWaiting();
}

async function deleteOldCaches() {
  for (const key of await self.caches.keys()) {
    if (key !== CACHE_KEY) self.caches.delete(key);
  }
}

async function getResponse(request) {
  const live_response = await fetch(request)
    .then((response) => {
      // Update the cache.
      cache.put(request, response.clone());
      return response;
    })
    .catch((error) => console.log("Backend offline"));
  if (live_response) {
    return live_response;
  }
  const cache = await self.caches.open(CACHE_KEY);
  const cached_response = await cache.match(request);
  return cached_response;
}

self.addEventListener("install", (event) => event.waitUntil(install()));

self.addEventListener("activate", (event) =>
  event.waitUntil(deleteOldCaches().then(self.clients.claim())),
);

self.addEventListener("fetch", (event) =>
  event.respondWith(getResponse(event.request)),
);
