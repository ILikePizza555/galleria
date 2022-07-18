import {Component, html, render} from 'https://unpkg.com/htm/preact/index.mjs?module';

/**
 * @typedef AppProps
 * @type {object}
 * @property {object[]} page_data 
 */

/**
 * @param {AppProps} props 
 */
function App(props) {
    if (props.page_data.length === 0) {
        return html`Looks like this gallery has no posts!`;
    } else {
        return html`
        <div class="gallery" role="list">
            ${props.page_data.map((post) => html`<${GalleryImage} ...${post} />`)}
        </div>
        `;
    }
}

/**
 * @typedef GalleryImageProps
 * @type {object}
 * @property {string?} source_url
 * @property {string?} media_url
 * @property {number?} media_width
 * @property {number?} media_height

 */

/**
 * @param {GalleryImageProps} props 
 */
function GalleryImage(props) {
    if (!props.media_url) {
        return html`<div class="error" role="listitem">Error loading this post</div>`;
    }

    let img_props = {
        rel: "noreferrer",
        loading: "lazy",
        src: props.media_url
    };

    if (props.media_width && props.media_width > 0) {
        img_props.width = props.media_width;
    }

    if (props.media_height && props.media_height > 0) {
        img_props.height = props.media_height;
    }

    let image = html`<img ...${img_props} />`;
    
    return html`<div class="gallery-item" role="listitem">
        ${props.source_url && html`
        <a href=${props.source_url} rel="noreferrer" target="_blank">
            ${image}
        </a>`}
        ${!props.source_url && image}
    </div>`
}

render(html`<${App} page_data=${page_data} />`, document.getElementById("app-container"));