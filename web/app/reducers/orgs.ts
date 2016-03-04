// Copyright:: Copyright (c) 2016 Chef Software, Inc.
//
// The terms of the Evaluation Agreement (Bldr) between Chef Software Inc. and the party accessing
// this file ("Licensee") apply to Licensee's use of the Software until such time that the Software
// is made available under an open source license such as the Apache 2.0 License.

import {Record} from "immutable";
import * as actionTypes from "../actions/index";
import initialState from "../initialState";

export default function orgs(state = initialState["orgs"], action) {
    switch (action.type) {
        case actionTypes.FINISH_CREATING_ORG:
            return state.mergeIn(
                ["beingCreated"], state.get("beingCreated").clear()
            ).setIn(["ui", "create", "saved"], false);

        case actionTypes.POPULATE_ORG:
            return state.mergeIn(["beingCreated"], action.payload).
                setIn(["ui", "create", "saved"], true).
                set("added",
                    state.get("added").concat(Record(action.payload)())
                );

        default:
            return state;
    }
}