/**
 * Copyright (c) 2022 Hemashushu <hippospark@gmail.com>, All rights reserved.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <stdio.h>

int main(int argc, char *argv[])
{
    if (argc == 0)
    {
        printf("no args");
    }
    else
    {
        for (int i = 0; argv[i] != 0; i++)
        {
            printf("%s|", argv[i]);
        }
    }

    return 0;
}