export const React = {
    createElement: function(tag: string, props: any, children: any) {
        let textNode = null;

        if (tag === "span" && typeof children["0"] === "string") {
            textNode = children["0"];
        }

        let id = __tsnat_createWidget(tag, textNode);

        if (props !== null && props.onClick) {
            __tsnat_addEventListener(id, props.onClick);
        }

        let i = 0;
        while (children[i] !== undefined) {
            let child = children[i];
            if (typeof child === "object" && child.id !== undefined) {
                __tsnat_appendChild(id, child.id);
            }
            i = i + 1;
        }

        return { id: id, tag: tag };
    }
};

export const ReactDOM = {
    render: function(root_component: any, root_id: any) {
        if (typeof root_component === "object" && root_component.id !== undefined) {
            __tsnat_setRoot(root_component.id);
        }
    }
};
