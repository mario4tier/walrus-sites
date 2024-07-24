// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { DomainDetails } from "./types/index";
import { getDomain } from "./domain_parsing";
import { aggregatorEndpoint } from "./aggregator";

/**
 * Redirects to the portal URL.
 */
export function redirectToPortalURLResponse(scope: URL, path: DomainDetails): Response {
    // Redirect to the walrus site for the specified domain and path
    const redirectUrl = getPortalUrl(path, scope.href);
    console.log("Redirecting to the Walrus Site link: ", path, redirectUrl);
    return makeRedirectResponse(redirectUrl);
}

/**
 * Redirects to the aggregator URL.
 */
export function redirectToAggregatorUrlResponse(scope: URL, blobId: string): Response {
    // Redirect to the walrus site for the specified domain and path
    const redirectUrl = aggregatorEndpoint(blobId);
    console.log("Redirecting to the Walrus Blob link: ", redirectUrl);
    return makeRedirectResponse(redirectUrl.href);
}

function makeRedirectResponse(url: string): Response {
    return new Response(null, {
        status: 302,
        headers: {
            Location: url,
        },
    });
}

/**
 * Returns the url for the Portal, given a subdomain and a path.
 */
function getPortalUrl(path: DomainDetails, scope: string): string {
    const scopeUrl = new URL(scope);
    const portalDomain = getDomain(scopeUrl);
    let portString = "";
    if (scopeUrl.port) {
        portString = ":" + scopeUrl.port;
    }
    return scopeUrl.protocol + "//" + path.subdomain + "." + portalDomain + portString + path.path;
}