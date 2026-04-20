let _state: any[] = [];
let _index = 0;
let _rootComponent: any = null;

export const React = {
    useState: function(initialValue: any) {
        const currentIndex = _index;
        if (_state[currentIndex] === undefined) {
            _state[currentIndex] = initialValue;
        }
        
        const setState = (newValue: any) => {
            if (typeof newValue === "function") {
                _state[currentIndex] = newValue(_state[currentIndex]);
            } else {
                _state[currentIndex] = newValue;
            }
            // Trigger Re-render
            _index = 0;
            if (_rootComponent !== null) {
                // We re-evaluate the component tree completely
                let newTree = _rootComponent();
                if (typeof newTree === "object" && newTree.id !== undefined) {
                    __tsnat_setRoot(newTree.id);
                }
            }
        };
        _index++;
        return [_state[currentIndex], setState];
    },

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

export const renderApp = function(root_fn: any, config: any) {
    _rootComponent = root_fn;
    _index = 0;
    
    let root_element = root_fn();
    if (typeof root_element === "object" && root_element.id !== undefined) {
        __tsnat_setRoot(root_element.id);
    }
};

export const ReactDOM = {
    render: function(root_component: any, root_id: any) {
        if (typeof root_component === "object" && root_component.id !== undefined) {
            __tsnat_setRoot(root_component.id);
        }
    }
};
