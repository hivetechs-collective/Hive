// Cloudflare Worker to serve R2 releases files
export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    
    // Remove leading slash from pathname
    const key = url.pathname.slice(1) || 'releases.json';
    
    try {
      // Debug logging
      console.log('Attempting to fetch key:', key);
      console.log('Bucket binding:', env.RELEASES_BUCKET ? 'exists' : 'missing');
      
      // Get the object from R2
      const object = await env.RELEASES_BUCKET.get(key);
      
      if (!object) {
        console.log('Object not found in bucket:', key);
        return new Response(`Not Found: ${key}`, { status: 404 });
      }
      
      // Set appropriate headers
      const headers = new Headers();
      object.writeHttpMetadata(headers);
      headers.set('etag', object.httpEtag);
      
      // Set content type based on file extension
      if (key.endsWith('.json')) {
        headers.set('content-type', 'application/json');
      } else if (key.endsWith('.html')) {
        headers.set('content-type', 'text/html');
      } else if (key.endsWith('.tar.gz')) {
        headers.set('content-type', 'application/gzip');
      } else if (key.endsWith('.exe')) {
        headers.set('content-type', 'application/octet-stream');
      } else {
        headers.set('content-type', 'application/octet-stream');
      }
      
      // Add CORS headers for cross-origin access
      headers.set('Access-Control-Allow-Origin', '*');
      headers.set('Access-Control-Allow-Methods', 'GET, HEAD, OPTIONS');
      
      return new Response(object.body, {
        headers,
      });
    } catch (error) {
      return new Response('Internal Server Error', { status: 500 });
    }
  },
};