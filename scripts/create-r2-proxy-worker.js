// Cloudflare Worker to serve R2 files publicly
export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    
    // Remove leading slash from pathname
    const key = url.pathname.slice(1);
    
    // Default to releases.json if no path
    const objectKey = key || 'releases.json';
    
    console.log('Fetching object:', objectKey);
    
    // Get the object from R2
    const object = await env.RELEASES_BUCKET.get(objectKey);
    
    if (!object) {
      return new Response('Object Not Found', { status: 404 });
    }
    
    // Set appropriate headers based on file type
    const headers = new Headers();
    object.writeHttpMetadata(headers);
    headers.set('etag', object.httpEtag);
    
    // Set content type based on file extension
    if (objectKey.endsWith('.json')) {
      headers.set('content-type', 'application/json');
    } else if (objectKey.endsWith('.html')) {
      headers.set('content-type', 'text/html');
    } else if (objectKey.endsWith('.tar.gz')) {
      headers.set('content-type', 'application/gzip');
    } else if (objectKey.endsWith('.exe')) {
      headers.set('content-type', 'application/octet-stream');
    }
    
    // Add CORS headers
    headers.set('Access-Control-Allow-Origin', '*');
    
    return new Response(object.body, {
      headers,
    });
  },
};