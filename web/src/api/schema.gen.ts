/**
 * This file was auto-generated by openapi-typescript.
 * Do not make direct changes to the file.
 */

export interface paths {
    "/playlists": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Get all playlists configured on the device. */
        get: {
            parameters: {
                query?: never;
                header?: never;
                path?: never;
                cookie?: never;
            };
            requestBody?: never;
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "application/json; charset=utf-8": components["schemas"]["PlaylistInfo"][];
                    };
                };
            };
        };
        put?: never;
        /** Create a new playlist */
        post: {
            parameters: {
                query?: never;
                header?: never;
                path?: never;
                cookie?: never;
            };
            requestBody: {
                content: {
                    "application/json; charset=utf-8": components["schemas"]["CreatePlaylistRequest"];
                };
            };
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "application/json; charset=utf-8": components["schemas"]["PlaylistInfo"];
                    };
                };
            };
        };
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/playlists/{playlist_id}/tabs": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Get all tabs from a given playlist. */
        get: {
            parameters: {
                query?: never;
                header?: never;
                path: {
                    playlist_id: string;
                };
                cookie?: never;
            };
            requestBody?: never;
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "application/json; charset=utf-8": components["schemas"]["TabInfo"][];
                    };
                };
            };
        };
        put?: never;
        /** Add a tab to a playlist */
        post: {
            parameters: {
                query?: never;
                header?: never;
                path: {
                    playlist_id: string;
                };
                cookie?: never;
            };
            requestBody: {
                content: {
                    "application/json; charset=utf-8": components["schemas"]["AddTabToPlaylistRequest"];
                };
            };
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "text/plain; charset=utf-8": string;
                    };
                };
            };
        };
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/status": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Retrieve basic device status information. */
        get: {
            parameters: {
                query?: never;
                header?: never;
                path?: never;
                cookie?: never;
            };
            requestBody?: never;
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "application/json; charset=utf-8": components["schemas"]["DeviceStatus"];
                    };
                };
            };
        };
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/playlists/{playlist_id}/activate": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Activate a playlist */
        post: {
            parameters: {
                query?: never;
                header?: never;
                path: {
                    playlist_id: string;
                };
                cookie?: never;
            };
            requestBody?: never;
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "text/plain; charset=utf-8": string;
                    };
                };
            };
        };
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/playlists/{playlist_id}/tabs/{tab_id}/activate": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Activate a tab immediately */
        post: {
            parameters: {
                query?: never;
                header?: never;
                path: {
                    playlist_id: string;
                    tab_id: string;
                };
                cookie?: never;
            };
            requestBody?: never;
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "text/plain; charset=utf-8": string;
                    };
                };
            };
        };
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/playlists/{playlist_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Update an existing playlist */
        put: {
            parameters: {
                query?: never;
                header?: never;
                path: {
                    playlist_id: string;
                };
                cookie?: never;
            };
            requestBody: {
                content: {
                    "application/json; charset=utf-8": components["schemas"]["UpdatePlaylistRequest"];
                };
            };
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "application/json; charset=utf-8": components["schemas"]["PlaylistInfo"];
                    };
                };
            };
        };
        post?: never;
        /** Delete a playlist */
        delete: {
            parameters: {
                query?: never;
                header?: never;
                path: {
                    playlist_id: string;
                };
                cookie?: never;
            };
            requestBody?: never;
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "text/plain; charset=utf-8": string;
                    };
                };
            };
        };
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/tabs": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Create a new tab */
        post: {
            parameters: {
                query?: never;
                header?: never;
                path?: never;
                cookie?: never;
            };
            requestBody: {
                content: {
                    "application/json; charset=utf-8": components["schemas"]["CreateTabRequest"];
                };
            };
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "application/json; charset=utf-8": components["schemas"]["TabInfo"];
                    };
                };
            };
        };
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/tabs/{tab_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Update an existing tab */
        put: {
            parameters: {
                query?: never;
                header?: never;
                path: {
                    tab_id: string;
                };
                cookie?: never;
            };
            requestBody: {
                content: {
                    "application/json; charset=utf-8": components["schemas"]["UpdateTabRequest"];
                };
            };
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "application/json; charset=utf-8": components["schemas"]["TabInfo"];
                    };
                };
            };
        };
        post?: never;
        /** Delete a tab */
        delete: {
            parameters: {
                query?: never;
                header?: never;
                path: {
                    tab_id: string;
                };
                cookie?: never;
            };
            requestBody?: never;
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "text/plain; charset=utf-8": string;
                    };
                };
            };
        };
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/playlists/{playlist_id}/tabs/{tab_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        post?: never;
        /** Remove a tab from a playlist */
        delete: {
            parameters: {
                query?: never;
                header?: never;
                path: {
                    playlist_id: string;
                    tab_id: string;
                };
                cookie?: never;
            };
            requestBody?: never;
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "text/plain; charset=utf-8": string;
                    };
                };
            };
        };
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/playlists/{playlist_id}/reorder": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        /** Reorder tabs in a playlist */
        put: {
            parameters: {
                query?: never;
                header?: never;
                path: {
                    playlist_id: string;
                };
                cookie?: never;
            };
            requestBody: {
                content: {
                    "application/json; charset=utf-8": components["schemas"]["ReorderTabsRequest"];
                };
            };
            responses: {
                200: {
                    headers: {
                        [name: string]: unknown;
                    };
                    content: {
                        "text/plain; charset=utf-8": string;
                    };
                };
            };
        };
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
}
export type webhooks = Record<string, never>;
export interface components {
    schemas: {
        /** AddTabToPlaylistRequest */
        AddTabToPlaylistRequest: {
            tab_id: string;
            /** Format: int64 */
            order_index: number;
            /** Format: int64 */
            duration_seconds?: number;
        };
        /** CreatePlaylistRequest */
        CreatePlaylistRequest: {
            id: string;
            name: string;
            /** Format: int64 */
            interval_seconds: number;
        };
        /** CreateTabRequest */
        CreateTabRequest: {
            id: string;
            name: string;
            url: string;
            persist?: boolean;
        };
        /**
         * DeviceStatus
         * @description Current device status
         */
        DeviceStatus: {
            /** @description Device unique identifier */
            device_id: string;
            /** @description Device display name */
            device_name: string;
            /** @description Currently active playlist ID (if any) */
            current_playlist?: string;
            /** @description Currently active tab ID (if any) */
            current_tab?: string;
            /**
             * Format: uint64
             * @description Uptime in seconds
             */
            uptime_seconds: number;
        };
        /**
         * PlaylistInfo
         * @description Information about a playlist
         */
        PlaylistInfo: {
            /** @description Unique identifier for the playlist */
            id: string;
            /** @description Display name of the playlist */
            name: string;
            /**
             * Format: uint64
             * @description Number of tabs in the playlist
             */
            tab_count: number;
            /**
             * Format: int64
             * @description Interval between tab switches in seconds
             */
            interval_seconds: number;
            /** @description Whether this playlist is currently active */
            is_active: boolean;
        };
        /** ReorderTabsRequest */
        ReorderTabsRequest: {
            tab_orders: components["schemas"]["TabOrder"][];
        };
        /**
         * TabInfo
         * @description Information about a tab
         */
        TabInfo: {
            /** @description Unique identifier for the tab */
            id: string;
            /** @description Display name of the tab */
            name: string;
            /** @description URL the tab displays */
            url: string;
            /**
             * Format: uint64
             * @description Order within the playlist (0-based index)
             */
            order_index: number;
            /** @description Whether this tab persists in browser memory */
            persist: boolean;
            /**
             * Format: int32
             * @description Viewport width in pixels (if available)
             */
            viewport_width?: number;
            /**
             * Format: int32
             * @description Viewport height in pixels (if available)
             */
            viewport_height?: number;
        };
        /** TabOrder */
        TabOrder: {
            tab_id: string;
            /** Format: int64 */
            order_index: number;
        };
        /** UpdatePlaylistRequest */
        UpdatePlaylistRequest: {
            name?: string;
            /** Format: int64 */
            interval_seconds?: number;
            is_active?: boolean;
        };
        /** UpdateTabRequest */
        UpdateTabRequest: {
            name?: string;
            url?: string;
            persist?: boolean;
        };
    };
    responses: never;
    parameters: never;
    requestBodies: never;
    headers: never;
    pathItems: never;
}
export type $defs = Record<string, never>;
export type operations = Record<string, never>;
