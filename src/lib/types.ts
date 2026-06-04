export enum cbEventType {
	Text = "text",
	Image = "image",
	File = "file",
}

export type clipboardEvent = {
	event_type: cbEventType;
	content: string;
	timestamp: string;
};

export type cbEventNotice = {
	event_type: cbEventType;
	timestamp: string;
};

// export type cbEventType = "text" | "image" | "file";
