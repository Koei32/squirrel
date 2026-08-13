export enum cbEventType {
	Text = "Text",
	Image = "Image",
	File = "File",
}

export type cbEventContent =
	| { type: cbEventType.Text; data: string | undefined }
	| { type: cbEventType.Image; data: Uint8Array<ArrayBufferLike> | undefined } // base64 encoded
	| { type: cbEventType.File; data: Array<string> | undefined }; // list of paths

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

export enum Theme {
	Light = "light",
	Dark = "dark",
	System = "system",
}

// export type Theme = "light" | "dark" | "system";
