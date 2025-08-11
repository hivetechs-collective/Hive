/**
 * Web Crypto API utilities for Cloudflare Workers
 */

// Convert string to ArrayBuffer
function stringToArrayBuffer(str) {
  return new TextEncoder().encode(str);
}

// Convert ArrayBuffer to hex string
function arrayBufferToHex(buffer) {
  return Array.from(new Uint8Array(buffer))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

// Create HMAC using Web Crypto API
export async function createHmac(algorithm, key, data) {
  const keyBuffer = stringToArrayBuffer(key);
  const dataBuffer = stringToArrayBuffer(data);
  
  const cryptoKey = await crypto.subtle.importKey(
    'raw',
    keyBuffer,
    { name: 'HMAC', hash: algorithm === 'sha256' ? 'SHA-256' : 'SHA-1' },
    false,
    ['sign']
  );
  
  const signature = await crypto.subtle.sign('HMAC', cryptoKey, dataBuffer);
  return arrayBufferToHex(signature);
}

// Create hash using Web Crypto API
export async function createHash(algorithm, data) {
  const buffer = stringToArrayBuffer(data);
  const hashAlgorithm = algorithm === 'sha256' ? 'SHA-256' : 'SHA-1';
  const hash = await crypto.subtle.digest(hashAlgorithm, buffer);
  return arrayBufferToHex(hash);
}

// Generate secure random bytes
export function randomBytes(length) {
  const array = new Uint8Array(length);
  crypto.getRandomValues(array);
  return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
}