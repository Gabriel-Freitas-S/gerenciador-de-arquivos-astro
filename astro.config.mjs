// @ts-check
import { defineConfig } from 'astro/config';
import minify from 'astro-min';

import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
    integrations: [
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
    vite: {
        plugins: [tailwindcss()],
    },
});