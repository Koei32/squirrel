export enum cbEventType {
	Text = "text",
	Image = "image",
	File = "file",
}

export type cbEventContent =
	| { type: "Text"; data: string | undefined }
	| { type: "Image"; data: Uint8Array<ArrayBufferLike> | undefined } // base64 encoded
	| { type: "File"; data: string | undefined }; // TODO

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
