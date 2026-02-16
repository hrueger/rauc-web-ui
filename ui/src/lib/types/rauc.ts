export interface RaucSlot {
	state: string;
	boot_status?: string;
	device: string;
	type: string;
	bootname: string;
	class: string;
	mountpoint?: string;
}

export interface RaucStatus {
	compatible: string;
	variant?: string;
	booted: string;
	boot_primary: string;
	slots: Record<string, RaucSlot>[];
	'artifact-repositories': unknown[];
}

export interface RaucBundleImage {
	filename?: string;
	type?: string;
	size?: number;
	checksum?: string;
}

export interface RaucBundleInfo {
	compatible: string;
	version: string;
	description?: string;
	build: string;
	format: string;
	hooks?: unknown[];
	hash?: string;
	images?: Record<string, RaucBundleImage>[];
}

export interface AppConfig {
	logo_url?: string;
	project_name: string;
	background_color: string;
	foreground_color: string;
	primary_color: string;
}
