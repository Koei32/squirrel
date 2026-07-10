export enum cbEventType {
	Text = "text",
	Image = "image",
	File = "file",
}

export type cbEventContent =
	| { type: "text"; data: string }
	| { type: "image"; data: string } // base64 encoded
	| { type: "file"; data: string }; // TODO

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
