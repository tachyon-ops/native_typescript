const React = {
    createElement: function(tag: string, props: any, child0: any) {
        let textNode = null;

        // If it's a generic text wrapper
        if (tag === "span" && typeof child0 === "string") {
            textNode = child0;
        }

        // Native Injection
        let id = __tsnat_createWidget(tag, textNode);

        if (typeof child0 === "object" && child0.id !== undefined) {
            __tsnat_appendChild(id, child0.id);
        }

        return { id: id, tag: tag };
    }
};

const ReactDOM = {
    render: function(root_component: any, root_id: any) {
        __tsnat_setRoot(root_component.id);
    }
};
