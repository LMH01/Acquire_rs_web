// Example POST method implementation:
// Copied from https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch
/**
 * Submits a post request to the url
 * @param {String} url The url to which the post request should be sent
 * @param {String} data Data formatted as json string
 * @param {Map} additional_headers Additional headers that should be added to the request
 * @returns 
 */
async function postData(url = '', data = {}, additional_headers) {
  const headers = new Headers;
  headers.append('Content-Type', 'application/json');
  for (const [key, value] of Object.entries(additional_headers)) {
    headers.append(key, value);
  }
  // Default options are marked with *
  const response = await fetch(url, {
    method: 'POST', // *GET, POST, PUT, DELETE, etc.
    mode: 'cors', // no-cors, *cors, same-origin
    cache: 'no-cache', // *default, no-cache, reload, force-cache, only-if-cached
    credentials: 'same-origin', // include, *same-origin, omit
    headers,
    redirect: 'follow', // manual, *follow, error
    referrerPolicy: 'no-referrer', // no-referrer, *no-referrer-when-downgrade, origin, origin-when-cross-origin, same-origin, strict-origin, strict-origin-when-cross-origin, unsafe-url
    body: JSON.stringify(data) // body data type must match "Content-Type" header
  });
  return response.json(); // parses JSON response into native JavaScript objects
}

/**
 * Submits a get request to the url
 * @param {String} url The url to which the get request should be sent
 * @returns The response
 */
async function fetchData(url = '') {
  var word = await (await(fetch(url, {}))).text();
  return word;
}

/**
 * Submits a get request to the url
 * @param {String} url The url to which the get request should be sent
 * @param {Map} additional_headers Headers that should be send
 * @returns The response
 */
async function fetchData(url, additional_headers) {
  const headers = new Headers;
  for (const [key, value] of additional_headers) {
    headers.append(key, value);
  }
  var response = await (fetch(url, {
    method: 'GET',
    headers,
  }));
  var json = await response.json();
  return json;
}