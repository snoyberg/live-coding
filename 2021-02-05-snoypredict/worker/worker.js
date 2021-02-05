addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

/**
 * Fetch and log a request
 * @param {Request} request
 */
async function handleRequest(request) {
    const { handle } = wasm_bindgen;
    await wasm_bindgen(wasm)
    const rust_response = await handle(request.url)
    let headers = new Headers()
    headers.append("Content-Type", rust_response.get_content_type())
    return new Response(rust_response.get_body(), {
      status: rust_response.get_status(),
      headers: headers
    })
}
