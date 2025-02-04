//
//  Request.swift
//  Signer
//
//  Created by Daniel Leping on 28/01/2023.
//

import Foundation

public enum Request: Equatable {
    case testSign(TestSign)
    case testError(TestError)
    case substrateAccount(SubstrateAccount)
    case substrateSign(SubstrateSign)
}
