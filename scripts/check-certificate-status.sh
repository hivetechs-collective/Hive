#!/bin/bash

echo "🔍 Checking Developer ID Certificate Status..."
echo "================================================"

IDENTITY="Developer ID Application: Verone Lazio (FWBLB27H52)"

# Check if certificate exists
echo -e "\n📄 Certificate Details:"
security find-certificate -c "$IDENTITY" -p > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ Certificate found in keychain"
    
    # Get certificate details
    CERT_INFO=$(security find-certificate -c "$IDENTITY" -p | openssl x509 -noout -subject -issuer -dates -fingerprint -sha1 2>/dev/null)
    echo "$CERT_INFO" | grep -E "subject=|issuer=|notBefore=|notAfter=|SHA1"
    
    # Check certificate validity
    CURRENT_TIME=$(date +%s)
    NOT_BEFORE=$(security find-certificate -c "$IDENTITY" -p | openssl x509 -noout -startdate | cut -d= -f2)
    NOT_AFTER=$(security find-certificate -c "$IDENTITY" -p | openssl x509 -noout -enddate | cut -d= -f2)
    NOT_BEFORE_EPOCH=$(date -j -f "%b %d %T %Y %Z" "$NOT_BEFORE" +%s 2>/dev/null || date -d "$NOT_BEFORE" +%s)
    NOT_AFTER_EPOCH=$(date -j -f "%b %d %T %Y %Z" "$NOT_AFTER" +%s 2>/dev/null || date -d "$NOT_AFTER" +%s)
    
    if [ $CURRENT_TIME -lt $NOT_BEFORE_EPOCH ]; then
        echo "❌ Certificate not yet valid!"
    elif [ $CURRENT_TIME -gt $NOT_AFTER_EPOCH ]; then
        echo "❌ Certificate has expired!"
    else
        echo "✅ Certificate is within validity period"
    fi
else
    echo "❌ Certificate not found in keychain"
    exit 1
fi

# Check private key
echo -e "\n🔐 Private Key Status:"
security find-identity -v -p codesigning | grep "$IDENTITY" > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ Private key found and paired with certificate"
else
    echo "❌ Private key not found or not paired"
fi

# Test signing
echo -e "\n📝 Signature Test:"
TEST_FILE=$(mktemp)
echo "test" > "$TEST_FILE"
codesign -s "$IDENTITY" "$TEST_FILE" 2>&1 | head -5
rm -f "$TEST_FILE"

# Check certificate chain
echo -e "\n🔗 Certificate Chain Status:"
if codesign -s "$IDENTITY" --dryrun /usr/bin/true 2>&1 | grep -q "unable to build chain"; then
    echo "⚠️  Certificate chain not yet recognized by Apple"
    echo "   This is normal for new certificates (24-48 hours typical)"
    
    # Calculate time since certificate creation
    CREATED=$(security find-certificate -c "$IDENTITY" -p | openssl x509 -noout -startdate | cut -d= -f2)
    CREATED_EPOCH=$(date -j -f "%b %d %T %Y %Z" "$CREATED" +%s 2>/dev/null || date -d "$CREATED" +%s)
    HOURS_SINCE=$((($CURRENT_TIME - $CREATED_EPOCH) / 3600))
    echo "   Time since certificate creation: ${HOURS_SINCE} hours"
    
    if [ $HOURS_SINCE -lt 24 ]; then
        echo "   🕰️ Status: Still within typical 24-hour window"
    elif [ $HOURS_SINCE -lt 48 ]; then
        echo "   🕰️ Status: Within 24-48 hour window"
    else
        echo "   ⚠️  Status: Exceeded typical 48-hour window"
        echo "   Consider contacting Apple Developer Support"
    fi
else
    echo "✅ Certificate chain is valid!"
    echo "   Your certificate is ready for signing"
fi

# OCSP check
echo -e "\n🌐 OCSP Status:"
OCSP_URI=$(security find-certificate -c "$IDENTITY" -p | openssl x509 -noout -ocsp_uri)
if [ -n "$OCSP_URI" ]; then
    echo "   OCSP URI: $OCSP_URI"
    # Note: Full OCSP check requires certificate chain which may not work yet
else
    echo "❌ No OCSP URI found"
fi

# Recommendations
echo -e "\n💡 Recommendations:"
if codesign -s "$IDENTITY" --dryrun /usr/bin/true 2>&1 | grep -q "unable to build chain"; then
    echo "1. Wait for Apple's infrastructure to recognize your certificate"
    echo "2. In the meantime, use ad-hoc signed builds:"
    echo "   ./scripts/create-adhoc-dmg.sh"
    echo "3. Check status again in a few hours:"
    echo "   ./scripts/check-certificate-status.sh"
    echo "4. Once ready, create the signed DMG:"
    echo "   ./scripts/create-signed-dmg.sh"
else
    echo "✅ Your certificate is ready!"
    echo "1. Create signed builds:"
    echo "   ./scripts/sign-macos.sh"
    echo "   ./scripts/create-signed-dmg.sh"
    echo "2. Consider setting up notarization for best user experience"
fi

echo -e "\n================================================"