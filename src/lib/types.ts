export enum cbEventType {
	Text = "text",
	Image = "image",
	File = "file",
}

export type clipboardEvent = {
	id: number;
	event_type: cbEventType;
	content: string;
	timestamp: string;
	is_pinned: boolean;
};

export type cbEventNotice = {
	id: number;
	event_type: cbEventType;
	timestamp: string;
};
