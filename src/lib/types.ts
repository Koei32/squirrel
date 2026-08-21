export enum CbEventType {
	Text = "Text",
	Image = "Image",
	File = "File",
}

export type CbEventContent =
	| { type: CbEventType.Text; data: string | undefined }
	| { type: CbEventType.Image; data: Uint8Array<ArrayBufferLike> | undefined } // base64 encoded
	| { type: CbEventType.File; data: Array<string> | undefined }; // list of paths

export type ClipboardEvent = {
	id: number;
	event_type: CbEventType;
	content: CbEventContent;
	is_pinned: boolean;
	expires_at: number;
};

export type Theme = "light" | "dark" | "system";
