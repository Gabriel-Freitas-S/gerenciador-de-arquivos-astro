// @ts-check
import { defineConfig } from 'astro/config';
import electron from 'astro-electron';
import minify from 'astro-min';

const nativeModules = ['@journeyapps/sqlcipher', 'sqlite3', 'mock-aws-s3', 'aws-sdk', 'nock'];

export default defineConfig({
	integrations: [
		electron({
			main: {
				entry: 'src/electron/main.ts',
				vite: {
					build: {
						commonjsOptions: {
							ignoreTryCatch: false,
						},
						rollupOptions: {
							external: nativeModules,
						},
					},
				},
			},
			preload: {
				input: 'src/electron/preload.ts',
			},
		}),
		minify({
			do_not_minify_doctype: false,
			ensure_spec_compliant_unquoted_attribute_values: true,
			keep_closing_tags: false,
			keep_comments: false,
			keep_html_and_head_opening_tags: false,
			keep_input_type_text_attr: false,
			keep_spaces_between_attributes: false,
			keep_ssi_comments: false,
			minify_css: true,
			minify_js: true,
			preserve_brace_template_syntax: true,
			preserve_chevron_percent_template_syntax: true,
			remove_bangs: true,
			remove_processing_instructions: true,
		}),
	],
});
