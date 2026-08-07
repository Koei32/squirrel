export enum cbEventType {
	Text = "Text",
	Image = "Image",
	Files = "Files",
}

export type cbEventContent =
	| { type: cbEventType.Text; data: string | undefined }
	| { type: cbEventType.Image; data: Uint8Array<ArrayBufferLike> | undefined } // base64 encoded
	| { type: cbEventType.Files; data: string | undefined }; // TODO

export type clipboardEvent = {
	id: number;
	event_type: cbEventType;
	content: cbEventContent;
	timestamp: string;
	is_pinned: boolean;
};

export type cbEventNotice = {
	id: number;
	event_type: cbEventType;
	timestamp: string;
};
